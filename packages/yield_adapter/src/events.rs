use soroban_sdk::{Address, Env, Symbol};

pub struct LendingAdapterEvents {}

impl LendingAdapterEvents {
    pub fn deposit(e: &Env, adapter: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "deposit"), adapter);
        e.events().publish(topics, (asset, amount));
    }

    pub fn withdraw(e: &Env, adapter: Address, user: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "withdraw"), adapter, user);
        e.events().publish(topics, (asset, amount));
    }

    pub fn claim_yield(e: &Env, adapter: Address, user: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "claim_yield"), adapter, user);
        e.events().publish(topics, (asset, amount));
    }

    pub fn claim_emissions(e: &Env, from: Address, to: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "emissions_claimed"), from);
        e.events().publish(topics, (to, asset, amount));
    }
    
    pub fn update_epoch_principal(e: &Env, asset: Address, epoch: u64, principal: i128) {
        let topics = (Symbol::new(e, "epoch_principal_updated"), asset);
        e.events().publish(topics, (epoch, principal));
    }
}
