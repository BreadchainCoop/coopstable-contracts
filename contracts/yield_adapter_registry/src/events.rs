use soroban_sdk::{Address, Env, Symbol};

pub struct YieldAdapterRegistryEvents {}

impl YieldAdapterRegistryEvents {
    
    pub fn set_admin(
        e: &Env,
        new_admin: Address,
    ) {
        let topics = (Symbol::new(e, "set_admin"), );
        e.events()
            .publish(topics, set_admin);
    }
    // TODO: Add events for register_adapter, remove_adapter, add_support_for_asset, remove_support_for_asset
}
