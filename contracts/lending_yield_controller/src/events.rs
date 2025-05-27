use soroban_sdk::{Address, Env, Symbol};
pub struct LendingYieldControllerEvents {}

impl LendingYieldControllerEvents {
    pub fn deposit_collateral(e: &Env, user: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "deposit_collateral"), user.clone());
        e.events().publish(topics, (asset, amount));
    }

    pub fn withdraw_collateral(e: &Env, user: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "withdraw_collateral"), user.clone());
        e.events().publish(topics, (asset, amount));
    }

    pub fn claim_yield(e: &Env, user: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "claim_yield"), user.clone());
        e.events().publish(topics, (asset, amount));
    }

    pub fn claim_emissions(e: &Env, user: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "claim_emissions"), user.clone());
        e.events().publish(topics, (asset, amount));
    }

    pub fn set_yield_distributor(e: &Env, yield_distributor: Address) { 
        let topics = (Symbol::new(e, "set_yield_distributor"), );
        e.events().publish(topics,  yield_distributor.clone());
    }

    pub fn set_adapter_registry(e: &Env, adapter_registry: Address) { 
        let topics = (Symbol::new(e, "set_adapter_registry"), );
        e.events().publish(topics,  adapter_registry.clone());
    }

    pub fn set_cusd_manager(e: &Env, cusd_manager: Address) { 
        let topics = (Symbol::new(e, "set_cusd_manager"), );
        e.events().publish(topics,  cusd_manager.clone());
    }
    
    pub fn set_admin(e: &Env, new_admin: Address) { 
        let topics = (Symbol::new(e, "set_admin"), );
        e.events().publish(topics,  new_admin.clone());
    }
}
