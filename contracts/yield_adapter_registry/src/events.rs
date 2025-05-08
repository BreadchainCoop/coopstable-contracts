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
        yield_type: Symbol,
        protocol: Symbol,
        adapter_address: Address,
    ) {
        let topics = (Symbol::new(e, "register_adapter"), yield_type);
        e.events()
            .publish(topics, (protocol, adapter_address));
    }
    
    pub fn remove_adapter(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        adapter_address: Address,
    ) {
        let topics = (Symbol::new(e, "remove_adapter"), yield_type);
        e.events()
            .publish(topics, (protocol, adapter_address));
    }
    
    pub fn add_support_for_asset(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        asset_address: Address,
    ) {
        let topics = (Symbol::new(e, "add_support_for_asset"), yield_type);
        e.events()
            .publish(topics, (protocol, asset_address));
    }
    
    pub fn remove_support_for_asset(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        asset_address: Address,
    ) {
        let topics = (Symbol::new(e, "remove_support_for_asset"), yield_type);
        e.events()
            .publish(topics, (protocol, asset_address));
    }
}