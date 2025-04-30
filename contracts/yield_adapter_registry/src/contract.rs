use soroban_sdk::{
    contract,  
    contractimpl,
    contractmeta,
    Address,
    Env,
};
use crate::{
    storage_types::{
        INSTANCE_BUMP_AMOUNT,
        INSTANCE_LIFETIME_THRESHOLD
    },
    storage::{
        register_yield_adapter, 
        remove_yield_adapter,
        get_yield_adapter,
        verify_if_yield_adapter_exists,
        support_asset,
        remove_asset_support,
        is_asset_supported
    },
    admin::{
        write_administrator,
        require_admin
    }
};
use yield_adapter::contract_types::SupportedAdapter;

contractmeta!(
    key = "Description",
    val = "Yield adapter registry for the Coopstable cUSD system"
);

pub trait YieldAdapterRegistryTrait {
    fn set_admin(env: &Env, new_admin: Address);
    fn register_adapter(env: &Env, protocol: SupportedAdapter, address: Address);
    fn get_adapter(env: &Env, protocol: SupportedAdapter) -> Address;
    fn remove_adapter(env: &Env, protocol: SupportedAdapter);
    fn add_support_for_asset(e: &Env, protocol: SupportedAdapter, asset_address: Address);
    fn remove_support_for_asset(e: &Env, protocol: SupportedAdapter, asset_address: Address);
    fn is_supported_asset(e: &Env, protocol: SupportedAdapter, asset_address: Address) -> bool;
}

#[contract]
pub struct YieldAdapterRegistry;

#[contractimpl]
impl YieldAdapterRegistry {
    fn __constructor(env: Env, admin: Address) {
        env.storage().instance().extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT
        );
        write_administrator(&env, &admin);
    }
}

#[contractimpl]
impl YieldAdapterRegistryTrait for YieldAdapterRegistry {
    fn set_admin(e: &Env, new_admin: Address) {
        require_admin(e);
        write_administrator(e, &new_admin);
        e.events().publish(("YIELD_ADAPTER_REGISTRY", "set_admin"), &new_admin);
    }

    fn register_adapter(
        e: &Env, 
        protocol: SupportedAdapter, 
        address: Address
    ) {        
        require_admin(e);
        register_yield_adapter(e, protocol.id(), address);
        e.events().publish(("YIELD_ADAPTER_REGISTRY", "register_adapter"), protocol.id());
    }

    fn remove_adapter(e: &Env, protocol: SupportedAdapter) {
        require_admin(e);
        remove_yield_adapter(e, protocol.id());
        e.events().publish(("YIELD_ADAPTER_REGISTRY", "remove_adapter"), protocol.id());
        
    }
    
    fn get_adapter(e: &Env, protocol: SupportedAdapter) -> Address {
        if verify_if_yield_adapter_exists(e, protocol.id()) {
            get_yield_adapter(e, protocol.id())
        } else {
            panic!("Yield adapter not found")
        }
    }

    fn add_support_for_asset(e: &Env, protocol: SupportedAdapter, asset_address: Address) {
        require_admin(e);
        support_asset(e, protocol.id(), asset_address.clone());
        e.events().publish(("YIELD_ADAPTER_REGISTRY", "add_support_for_asset"), asset_address);
    }

    fn remove_support_for_asset(e: &Env, protocol: SupportedAdapter, asset_address: Address) {
        require_admin(&e);
        remove_asset_support(&e, protocol.id(), asset_address.clone());
        e.events().publish(("CUSD_MANAGER", "remove_asset"), asset_address);
    }

    fn is_supported_asset(e: &Env, protocol: SupportedAdapter, asset_address: Address) -> bool {
        is_asset_supported(e, protocol.id(), asset_address)
    }
}
