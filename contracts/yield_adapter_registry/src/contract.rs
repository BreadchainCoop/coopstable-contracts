use soroban_sdk::{
    contract,  
    contractimpl,
    contractmeta,
    Address,
    Env,
};
use crate::storage::{
        register_yield_adapter, 
        remove_yield_adapter,
        get_yield_adapter,
        verify_if_yield_adapter_exists,
        support_asset,
        remove_asset_support,
        is_asset_supported
    };

use access_control::{
    access::default_access_control,
    constants::DEFAULT_ADMIN_ROLE
};
use yield_adapter::contract_types::SupportedAdapter;

contractmeta!(
    key = "Description",
    val = "Yield adapter registry for the Coopstable cUSD system"
);

pub trait YieldAdapterRegistryTrait {
    fn set_admin(e: &Env, caller: Address, new_admin: Address);
    fn register_adapter(e: &Env, caller: Address, protocol: SupportedAdapter, adapter_address: Address);
    fn get_adapter(e: &Env, protocol: SupportedAdapter) -> Address;
    fn remove_adapter(e: &Env, caller: Address, protocol: SupportedAdapter);
    fn add_support_for_asset(e: &Env, caller: Address, protocol: SupportedAdapter, asset_address: Address);
    fn remove_support_for_asset(e: &Env, caller: Address, protocol: SupportedAdapter, asset_address: Address);
    fn is_supported_asset(e: &Env, protocol: SupportedAdapter, asset_address: Address) -> bool;
}

#[contract]
pub struct YieldAdapterRegistry;

#[contractimpl]
impl YieldAdapterRegistry {
    fn __constructor(e: Env, admin: Address) {

        let access_control = default_access_control(&e);

        access_control.initialize(&e, &admin);
        access_control.set_role_admin(&e, DEFAULT_ADMIN_ROLE, DEFAULT_ADMIN_ROLE); 
        access_control._grant_role(&e, DEFAULT_ADMIN_ROLE, &admin);
    }
    fn only_admin(e: &Env, caller: Address) {
        let access_control = default_access_control(e);
        access_control.only_role(&e, &caller, DEFAULT_ADMIN_ROLE);
    }
}

#[contractimpl]
impl YieldAdapterRegistryTrait for YieldAdapterRegistry {
    fn set_admin(e: &Env, caller: Address, new_admin: Address) {
        let access_control = default_access_control(e);
        access_control.grant_role(&e, caller, DEFAULT_ADMIN_ROLE, &new_admin);
        e.events().publish(("YIELD_ADAPTER_REGISTRY", "set_admin"), &new_admin);
    }

    fn register_adapter(
        e: &Env, 
        caller: Address,
        protocol: SupportedAdapter,
        adapter_address: Address
    ) {        
        Self::only_admin(e, caller);
        register_yield_adapter(e, protocol.id(), adapter_address);
        e.events().publish(("YIELD_ADAPTER_REGISTRY", "register_adapter"), protocol.id());
    }

    fn remove_adapter(e: &Env, caller: Address, protocol: SupportedAdapter) {
        Self::only_admin(e, caller);
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

    fn add_support_for_asset(e: &Env, caller: Address, protocol: SupportedAdapter, asset_address: Address) {
        Self::only_admin(e, caller);
        support_asset(e, protocol.id(), asset_address.clone());
        e.events().publish(("YIELD_ADAPTER_REGISTRY", "add_support_for_asset"), asset_address);
    }

    fn remove_support_for_asset(e: &Env, caller: Address, protocol: SupportedAdapter, asset_address: Address) {
        Self::only_admin(e, caller);
        remove_asset_support(&e, protocol.id(), asset_address.clone());
        e.events().publish(("CUSD_MANAGER", "remove_asset"), asset_address);
    }

    fn is_supported_asset(e: &Env, protocol: SupportedAdapter, asset_address: Address) -> bool {
        is_asset_supported(e, protocol.id(), asset_address)
    }
    
}
