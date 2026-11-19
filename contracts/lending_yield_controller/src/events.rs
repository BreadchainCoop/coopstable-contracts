use soroban_sdk::{Address, BytesN, Env, Symbol};
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

    pub fn harvest_yield(e: &Env, protocol: Symbol, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "harvest_yield"), protocol);
        e.events().publish(topics, (asset, amount));
    }

    pub fn recompound_yield(e: &Env, protocol: Symbol, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "recompound_yield"), protocol);
        e.events().publish(topics, (asset, amount));
    }

    pub fn finalize_distribution(e: &Env, protocol: Symbol, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "finalize_distribution"), protocol);
        e.events().publish(topics, (asset, amount));
    }

    pub fn cancel_harvest(e: &Env, protocol: Symbol, asset: Address) {
        let topics = (Symbol::new(e, "cancel_harvest"), protocol);
        e.events().publish(topics, asset);
    }

    pub fn recollateralize(e: &Env, protocol: Symbol, asset: Address, from: Address, amount: i128) {
        let topics = (Symbol::new(e, "recollateralize"), protocol);
        e.events().publish(topics, (asset, from, amount));
    }

    pub fn rescue_collateral(e: &Env, protocol: Symbol, asset: Address, to: Address, amount: i128) {
        let topics = (Symbol::new(e, "rescue_collateral"), protocol);
        e.events().publish(topics, (asset, to, amount));
    }

    pub fn rescue(e: &Env, asset: Address, to: Address, amount: i128) {
        let topics = (Symbol::new(e, "rescue"),);
        e.events().publish(topics, (asset, to, amount));
    }

    pub fn paused(e: &Env, mint: bool, redeem: bool) {
        let topics = (Symbol::new(e, "paused"),);
        e.events().publish(topics, (mint, redeem));
    }

    pub fn upgrade(e: &Env, new_wasm_hash: BytesN<32>) {
        let topics = (Symbol::new(e, "upgrade"),);
        e.events().publish(topics, new_wasm_hash);
    }
}
