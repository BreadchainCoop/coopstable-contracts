use soroban_sdk::{Address, Env, Symbol};

pub struct YieldAdapterRegistryEvents {}

impl YieldAdapterRegistryEvents {
    
    pub fn set_admin(
        e: &Env,
        new_admin: Address,
    ) {
        let topics = (Symbol::new(e, "set_admin"), );
        e.events()
            .publish(topics, new_admin);
    }
    
    pub fn register_adapter(
        e: &Env,
        protocol: Symbol,
        adapter_address: Address,
    ) {
        let topics = (Symbol::new(e, "register_adapter"), protocol);
        e.events()
            .publish(topics, adapter_address);
    }
    
    pub fn remove_adapter(
        e: &Env,
        protocol: Symbol,
    ) {
        let topics = (Symbol::new(e, "remove_adapter"), );
        e.events()
            .publish(topics, protocol);
    }
    
    pub fn add_support_for_asset(
        e: &Env,
        protocol: Symbol,
        asset_address: Address,
    ) {
        let topics = (Symbol::new(e, "add_support_for_asset"), protocol);
        e.events()
            .publish(topics, asset_address);
    }
    
    pub fn remove_support_for_asset(
        e: &Env,
        protocol: Symbol,
        asset_address: Address,
    ) {
        let topics = (Symbol::new(e, "remove_support_for_asset"), protocol);
        e.events()
            .publish(topics, asset_address);
    }
}