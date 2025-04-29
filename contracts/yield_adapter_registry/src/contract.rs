use soroban_sdk::{
    contract,  
    contractimpl,
    contractmeta, 
    Symbol,
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
        verify_if_yield_adapter_exists
    },
    admin::{
        write_administrator,
        require_admin
    }
};

contractmeta!(
    key = "Description",
    val = "Yield adapter registry for the Coopstable cUSD system"
);

pub trait YieldAdapterRegistryTrait {
    fn set_admin(env: &Env, new_admin: Address);
    fn register_adapter(env: &Env, protocol: Symbol, address: Address);
    fn get_adapter(env: &Env, protocol: Symbol) -> Address;
    fn remove_adapter(env: &Env, protocol: Symbol);
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
        protocol: Symbol, 
        address: Address
    ) {        
        require_admin(e);
        register_yield_adapter(e, protocol.clone(), address);
        e.events().publish(("YIELD_ADAPTER_REGISTRY", "register_adapter"), &protocol);
    }

    fn remove_adapter(e: &Env, protocol: Symbol) {
        require_admin(e);
        remove_yield_adapter(e, protocol.clone());
    }

    fn get_adapter(e: &Env, protocol: Symbol) -> Address {
        verify_if_yield_adapter_exists(e, protocol.clone());
        get_yield_adapter(e, protocol)
    }
}
