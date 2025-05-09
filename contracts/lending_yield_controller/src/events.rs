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
}
