use soroban_sdk::{Address, Env, Symbol};

pub struct LendingAdapterEvents {}

impl LendingAdapterEvents {
    pub fn deposit(e: &Env, adapter: Address, user: Address, asset: Address, amount: i128) {
        let topics = (Symbol::new(e, "deposit"), adapter, user);
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
}
